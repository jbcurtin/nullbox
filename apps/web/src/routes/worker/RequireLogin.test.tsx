import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { clearSession, getSession } from "../../api/session";
import { CopyProvider } from "../../copy/CopyProvider";
import { RequireLogin } from "./RequireLogin";

function renderInbox() {
  const router = createMemoryRouter(
    [{ element: <RequireLogin />, children: [{ path: "/inbox", element: <p>secret inbox</p> }] }],
    { initialEntries: ["/inbox"] },
  );
  render(
    <CopyProvider locale="en">
      <QueryClientProvider client={new QueryClient()}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </CopyProvider>,
  );
}

function reply(status: number, body: unknown) {
  return Promise.resolve(new Response(JSON.stringify(body), { status }));
}

/** The form's submit button (the "Log in" tab has the same name). */
function submit() {
  const button = document.querySelector<HTMLButtonElement>("form button[type=submit]");
  if (!button) throw new Error("no submit button");
  return userEvent.click(button);
}

function body(call: unknown[]) {
  return JSON.parse(String((call[1] as RequestInit).body));
}

describe("RequireLogin", () => {
  beforeEach(() => clearSession());
  afterEach(() => vi.unstubAllGlobals());

  it("hides the inbox until the worker logs in", async () => {
    const fetch = vi.fn((_url: string, _init?: RequestInit) =>
      reply(200, { token: "tok", username: "agent", expires_at: "2999-01-01T00:00:00Z" }),
    );
    vi.stubGlobal("fetch", fetch);
    renderInbox();

    expect(screen.queryByText("secret inbox")).toBeNull();
    expect(screen.getByText(/Each account can log in once/)).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("Username"), "agent");
    await userEvent.type(screen.getByLabelText("Password"), "correct horse");
    await submit();

    expect(await screen.findByText("secret inbox")).toBeInTheDocument();
    expect(getSession()?.token).toBe("tok");
    expect(fetch.mock.calls[0]?.[0]).toMatch(/\/api\/auth\/login$/);
    expect(body(fetch.mock.calls[0] ?? [])).toEqual({
      username: "agent",
      password: "correct horse",
    });
  });

  it("says so when the password is wrong", async () => {
    vi.stubGlobal("fetch", () =>
      reply(401, { code: "invalid_credentials", message: "wrong username or password" }),
    );
    renderInbox();

    await userEvent.type(screen.getByLabelText("Username"), "agent");
    await userEvent.type(screen.getByLabelText("Password"), "nope");
    await submit();

    expect(await screen.findByRole("alert")).toHaveTextContent("Wrong username or password.");
    expect(screen.queryByText("secret inbox")).toBeNull();
    expect(getSession()).toBeNull();
  });

  it("tells a returning worker to create a new account", async () => {
    vi.stubGlobal("fetch", () =>
      reply(403, { code: "account_already_used", message: "already used" }),
    );
    renderInbox();

    await userEvent.type(screen.getByLabelText("Username"), "agent");
    await userEvent.type(screen.getByLabelText("Password"), "correct horse");
    await submit();

    expect(await screen.findByRole("alert")).toHaveTextContent("please create a new one");
    expect(getSession()).toBeNull();
  });

  it("creates an account, then logs in with it", async () => {
    const fetch = vi.fn((url: string, _init?: RequestInit) => {
      if (url.endsWith("/auth/options"))
        return reply(200, { signup_code_required: true, min_password_length: 12 });
      if (url.endsWith("/auth/signup")) return reply(201, { username: "aiko" });
      return reply(200, { token: "tok", username: "aiko", expires_at: "2999-01-01T00:00:00Z" });
    });
    vi.stubGlobal("fetch", fetch);
    renderInbox();

    await userEvent.click(screen.getByRole("button", { name: "Create account" }));
    await userEvent.type(screen.getByLabelText("Username"), "aiko");
    await userEvent.type(screen.getByLabelText("Password"), "violet otter gravel 42");
    await userEvent.type(await screen.findByLabelText("Sign-up code"), "team-07");
    await submit();

    // Back on the login tab, with the new username filled in.
    expect(
      await screen.findByText("Account “aiko” created. Log in to use it."),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Username")).toHaveValue("aiko");
    const signup = fetch.mock.calls.find(([url]) => url.endsWith("/auth/signup")) ?? [];
    expect(body(signup)).toEqual({
      username: "aiko",
      password: "violet otter gravel 42",
      signup_code: "team-07",
    });

    await userEvent.type(screen.getByLabelText("Password"), "violet otter gravel 42");
    await submit();
    expect(await screen.findByText("secret inbox")).toBeInTheDocument();
  });
});
