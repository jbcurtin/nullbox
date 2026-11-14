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

describe("RequireLogin", () => {
  beforeEach(() => clearSession());
  afterEach(() => vi.unstubAllGlobals());

  it("hides the inbox until the worker logs in", async () => {
    const fetch = vi.fn(() =>
      reply(200, { token: "tok", username: "agent", expires_at: "2999-01-01T00:00:00Z" }),
    );
    vi.stubGlobal("fetch", fetch);
    renderInbox();

    expect(screen.queryByText("secret inbox")).toBeNull();
    await userEvent.type(screen.getByLabelText("Username"), "agent");
    await userEvent.type(screen.getByLabelText("Password"), "correct horse");
    await userEvent.click(screen.getByRole("button", { name: "Log in" }));

    expect(await screen.findByText("secret inbox")).toBeInTheDocument();
    expect(getSession()?.token).toBe("tok");
    const [url, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toMatch(/\/api\/auth\/login$/);
    expect(JSON.parse(String(init.body))).toEqual({ username: "agent", password: "correct horse" });
  });

  it("says so when the password is wrong", async () => {
    vi.stubGlobal("fetch", () =>
      reply(401, { code: "invalid_credentials", message: "wrong username or password" }),
    );
    renderInbox();

    await userEvent.type(screen.getByLabelText("Username"), "agent");
    await userEvent.type(screen.getByLabelText("Password"), "nope");
    await userEvent.click(screen.getByRole("button", { name: "Log in" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Wrong username or password.");
    expect(screen.queryByText("secret inbox")).toBeNull();
    expect(getSession()).toBeNull();
  });
});
