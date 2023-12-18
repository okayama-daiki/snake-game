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
    <div
      className={styles.container}
      onClick={() => {
        connectionStatus === ConnectionStatus.OPEN && toGame();
      }}
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
    </div>
  );
}

const SnakeLoader = () => {
  return (
    <span className={styles.loader}>
      {Array(4)
        .fill(null)
        .map((_, i) => {
          return <span key={i} className={styles.cell}></span>;
        })}
    </span>
  );
};
