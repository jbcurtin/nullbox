import { Link, useSearchParams } from "react-router";
import { useCheckout } from "../../api/queries";
import { ErrorMessage } from "../../components/Feedback";
import { useCopy } from "../../copy/CopyProvider";
import styles from "../pages.module.css";

/** The shop: one product, paid for on Stripe's own checkout page. */
export function ShopPage() {
  const copy = useCopy();
  const checkout = useCheckout();
  // Still "pending" while the browser leaves for Stripe, so the button can't be pressed twice.
  const leaving = checkout.isPending || checkout.isSuccess;

  return (
    <section className={styles.page}>
      <div>
        <h1>{copy.shop.title}</h1>
        <p className={styles.intro}>{copy.shop.intro}</p>
      </div>
      <article className={`${styles.card} ${styles.page}`}>
        <h2>{copy.shop.productName}</h2>
        <p>{copy.shop.productDescription}</p>
        <p className={styles.stat}>{copy.shop.price}</p>
        {checkout.error && <ErrorMessage error={checkout.error} />}
        <div>
          <button
            type="button"
            disabled={leaving}
            onClick={() =>
              checkout.mutate(undefined, { onSuccess: ({ url }) => window.location.assign(url) })
            }
          >
            {leaving ? copy.shop.redirecting : copy.shop.buy}
          </button>
        </div>
        <p className={styles.hint}>{copy.shop.paymentHint}</p>
      </article>
    </section>
  );
}

/** Where Stripe sends the buyer after paying. */
export function ShopThanksPage() {
  const copy = useCopy();
  const [params] = useSearchParams();
  const session = params.get("session_id");

  return (
    <section className={`${styles.card} ${styles.page}`}>
      <h1>{copy.shop.thanksTitle}</h1>
      <p>{copy.shop.thanksBody}</p>
      {session && (
        <p className={styles.hint}>
          {copy.shop.reference}: <code>{session}</code>
        </p>
      )}
      <div>
        <Link to="/">← {copy.app.nav.submit}</Link>
      </div>
    </section>
  );
}
