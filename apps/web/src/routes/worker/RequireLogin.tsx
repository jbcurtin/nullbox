import { useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useState, useSyncExternalStore } from "react";
import { Outlet } from "react-router";
import { api } from "../../api/client";
import { getSession, setSession, subscribeSession } from "../../api/session";
import { ErrorMessage } from "../../components/Feedback";
import { useCopy } from "../../copy/CopyProvider";
import styles from "../pages.module.css";

/** Every worker route sits behind this: no session, no inbox. */
export function RequireLogin() {
  const session = useSyncExternalStore(subscribeSession, getSession);
  return session ? <Outlet /> : <LoginPage />;
}

type Mode = "login" | "signup";

/**
 * Log in, or create an account. Each account logs in once (the server enforces it),
 * so after logging out a worker comes back here and creates a new one.
 */
export function LoginPage() {
  const copy = useCopy();
  const [mode, setMode] = useState<Mode>("login");
  const [created, setCreated] = useState<string | null>(null);

  return (
    <section className={styles.page}>
      <div>
        <h1>{copy.login.title}</h1>
        <p className={styles.intro}>{copy.login.intro}</p>
      </div>
      <p className={`${styles.warning} ${styles.loginForm}`}>{copy.login.oneTimeNotice}</p>
      <div className={styles.tabs}>
        <button type="button" aria-pressed={mode === "login"} onClick={() => setMode("login")}>
          {copy.login.tabLogin}
        </button>
        <button type="button" aria-pressed={mode === "signup"} onClick={() => setMode("signup")}>
          {copy.login.tabSignup}
        </button>
      </div>
      {mode === "login" ? (
        <LoginForm createdUsername={created} />
      ) : (
        <SignupForm
          onCreated={(username) => {
            setCreated(username);
            setMode("login");
          }}
        />
      )}
    </section>
  );
}

function LoginForm({ createdUsername }: { createdUsername: string | null }) {
  const copy = useCopy();
  const queryClient = useQueryClient();
  const [error, setError] = useState<unknown>(null);
  const [pending, setPending] = useState(false);

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    setPending(true);
    setError(null);
    try {
      const response = await api.login({
        username: String(form.get("username") ?? "").trim(),
        password: String(form.get("password") ?? ""),
      });
      // Drop anything fetched (and refused) before logging in.
      queryClient.removeQueries({ queryKey: ["tickets"] });
      queryClient.removeQueries({ queryKey: ["backlog"] });
      setSession(response);
    } catch (err) {
      setError(err);
      setPending(false);
    }
  };

  return (
    <form className={`${styles.card} ${styles.form} ${styles.loginForm}`} onSubmit={onSubmit}>
      {createdUsername && (
        <p className={styles.hint} role="status">
          {copy.login.created(createdUsername)}
        </p>
      )}
      <label className={styles.field}>
        {copy.login.username}
        <input
          name="username"
          required
          autoComplete="username"
          maxLength={32}
          defaultValue={createdUsername ?? ""}
          key={createdUsername ?? ""}
        />
      </label>
      <label className={styles.field}>
        {copy.login.password}
        <input name="password" type="password" required autoComplete="current-password" />
      </label>
      {error !== null && <ErrorMessage error={error} />}
      <div>
        <button type="submit" disabled={pending}>
          {pending ? copy.login.submitting : copy.login.submit}
        </button>
      </div>
    </form>
  );
}

function SignupForm({ onCreated }: { onCreated: (username: string) => void }) {
  const copy = useCopy();
  const options = useQuery({ queryKey: ["auth-options"], queryFn: api.authOptions });
  const minLength = options.data?.min_password_length ?? 12;
  const [error, setError] = useState<unknown>(null);
  const [pending, setPending] = useState(false);

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const code = String(form.get("signup_code") ?? "").trim();
    setPending(true);
    setError(null);
    try {
      const response = await api.signup({
        username: String(form.get("username") ?? "").trim(),
        password: String(form.get("password") ?? ""),
        signup_code: code || undefined,
      });
      onCreated(response.username);
    } catch (err) {
      setError(err);
      setPending(false);
    }
  };

  return (
    <form className={`${styles.card} ${styles.form} ${styles.loginForm}`} onSubmit={onSubmit}>
      <label className={styles.field}>
        {copy.login.username}
        <input
          name="username"
          required
          autoComplete="username"
          minLength={3}
          maxLength={32}
          pattern="[\p{L}\p{N}_.\-]+"
          aria-describedby="signup-username-hint"
        />
      </label>
      <span id="signup-username-hint" className={styles.hint}>
        {copy.login.usernameHint}
      </span>
      <label className={styles.field}>
        {copy.login.password}
        <input
          name="password"
          type="password"
          required
          minLength={minLength}
          autoComplete="new-password"
          aria-describedby="signup-password-hint"
        />
      </label>
      <span id="signup-password-hint" className={styles.hint}>
        {copy.login.passwordHint(minLength)}
      </span>
      {options.data?.signup_code_required && (
        <>
          <label className={styles.field}>
            {copy.login.signupCode}
            <input
              name="signup_code"
              required
              autoComplete="off"
              aria-describedby="signup-code-hint"
            />
          </label>
          <span id="signup-code-hint" className={styles.hint}>
            {copy.login.signupCodeHint}
          </span>
        </>
      )}
      {error !== null && <ErrorMessage error={error} />}
      <div>
        <button type="submit" disabled={pending}>
          {pending ? copy.login.signupSubmitting : copy.login.signupSubmit}
        </button>
      </div>
    </form>
  );
}
