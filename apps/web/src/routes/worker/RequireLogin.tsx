import { useQueryClient } from "@tanstack/react-query";
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

export function LoginPage() {
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
    <section className={styles.page}>
      <div>
        <h1>{copy.login.title}</h1>
        <p className={styles.intro}>{copy.login.intro}</p>
      </div>
      <form className={`${styles.card} ${styles.form} ${styles.loginForm}`} onSubmit={onSubmit}>
        <label className={styles.field}>
          {copy.login.username}
          <input name="username" required autoComplete="username" maxLength={100} />
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
    </section>
  );
}
