import styles from "./Error.module.scss";

export default function ErrorModal() {
  return (
    <dialog className={styles.modal}>
      <div className={styles.content}>
        <div className={styles.header}>
          <img
            src="/snake-game/error.webp"
            alt="error icon"
            width={150}
            height={150}
            className={styles.errorIcon}
          />
          <h1 className={styles.title}>Technical Problem</h1>
        </div>
        <p className={styles.description}>
          Snake Game is unavailable at the moment, due to a technical problem.
          Please refresh the page or try again later.
        </p>
        <button
          className={styles.button}
          onClick={() => window.location.reload()}
        >
          Reload Page
        </button>
      </div>
    </dialog>
  );
}
