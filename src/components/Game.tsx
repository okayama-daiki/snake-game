import { useEffect, useRef } from "react";
import { RenderEngine } from "../services/renderer/pkg";
import type { RankingEntry } from "../types";
import styles from "./Game.module.scss";
import Leaderboard from "./Leaderboard";

export default function Game({
  socket,
  toLobby,
  ranking,
  playerToken,
}: {
  socket: WebSocket;
  toLobby: () => void;
  ranking: RankingEntry[];
  playerToken: string;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) return;
    const canvas = canvasRef.current;

    const engine = new RenderEngine(canvas, socket, toLobby, playerToken);
    engine.init();

    return () => engine.destroy();
  }, [socket, toLobby, playerToken]);

  return (
    <div className={styles.container}>
      <Leaderboard entries={ranking} />
      <canvas ref={canvasRef} className={styles.canvas}></canvas>
    </div>
  );
}
