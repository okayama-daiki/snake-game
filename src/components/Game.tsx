import { useEffect, useRef } from "react";
import GameEngine from "../services/GameEngine";
import styles from "./Game.module.scss";

const URI = "ws://0.0.0.0:8000";

export default function Game({ toLobby }: { toLobby: () => void }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) return;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    new GameEngine(canvas, ctx, URI);

    toLobby;
  }, [canvasRef]);

  return (
    <div className={styles.container}>
      <canvas ref={canvasRef} className={styles.canvas}></canvas>
    </div>
  );
}
