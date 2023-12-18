import { useEffect, useRef } from "react";
import GameEngine from "../services/GameEngine";
import styles from "./Game.module.scss";

export default function Game({
  socket,
  toLobby,
}: {
  socket: WebSocket;
  toLobby: () => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) return;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    new GameEngine(canvas, ctx, socket);

    toLobby;
  }, [canvasRef]);

  return (
    <div className={styles.container}>
      <canvas ref={canvasRef} className={styles.canvas}></canvas>
    </div>
  );
}
