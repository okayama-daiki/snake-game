import { ConnectionStatus } from "../types";
import styles from "./Lobby.module.scss";

export default function Lobby({
  connectionStatus,
  toGame,
}: {
  connectionStatus: ConnectionStatus;
  toGame: () => void;
}) {
  const canStart = connectionStatus === ConnectionStatus.OPEN;
  const startGame = () => {
    if (canStart) toGame();
  };

  return (
    // biome-ignore lint/a11y/useSemanticElements: Preserve the original full-screen lobby styling while retaining keyboard support.
    <div
      className={styles.container}
      role="button"
      tabIndex={canStart ? 0 : -1}
      aria-disabled={!canStart}
      onClick={startGame}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          startGame();
        }
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
