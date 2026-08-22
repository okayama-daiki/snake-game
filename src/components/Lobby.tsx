import { ConnectionStatus } from "../types";
import styles from "./Lobby.module.scss";

export default function Lobby({
  connectionStatus,
  toGame,
}: {
  connectionStatus: ConnectionStatus;
  toGame: () => void;
}) {
  return (
    <button
      type="button"
      className={styles.container}
      disabled={connectionStatus !== ConnectionStatus.OPEN}
      onClick={toGame}
    >
      <h1 className={styles.title}>Snake Game</h1>
      <p className={styles.message}>
        {connectionStatus === ConnectionStatus.CONNECTING && (
          <>
            <span className={styles.fadeIn}>Connecting</span>
            <SnakeLoader />
          </>
        )}
        {connectionStatus === ConnectionStatus.OPEN && (
          <span className={styles.blink}>Tap to Start</span>
        )}
      </p>
    </button>
  );
}

const loaderCells = ["head", "body-1", "body-2", "tail"];

const SnakeLoader = () => {
  return (
    <span className={styles.loader}>
      {loaderCells.map((cell) => {
        return <span key={cell} className={styles.cell}></span>;
      })}
    </span>
  );
};
