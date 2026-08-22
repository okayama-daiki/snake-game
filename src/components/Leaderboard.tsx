import type { RankingEntry } from "../types";
import styles from "./Leaderboard.module.scss";

export default function Leaderboard({ entries }: { entries: RankingEntry[] }) {
  return (
    <aside className={styles.root} aria-label="Leaderboard">
      <ol className={styles.list}>
        {entries.map((entry) => (
          <li
            className={`${styles.entry} ${entry.rank > 10 ? styles.outside : ""}`}
            key={`${entry.is_bot ? "bot" : "player"}-${entry.name}`}
          >
            <span className={styles.rank}>{entry.rank}.</span>
            <span className={styles.name}>
              {entry.is_self ? "You" : entry.name}
            </span>
            <span className={styles.score}>{entry.score}</span>
          </li>
        ))}
      </ol>
    </aside>
  );
}
