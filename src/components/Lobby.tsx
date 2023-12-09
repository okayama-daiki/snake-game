import styles from "./Lobby.module.scss";

export default function Lobby({ toGame }: { toGame: () => void }) {
  return (
    <div className={styles.container} onClick={toGame}>
      <h1 className={styles.title}>Snake Game</h1>
      <p className={styles.description}>Tap to Start</p>
    </div>
  );
}
