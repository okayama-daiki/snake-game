import { useEffect, useRef } from "react";
import { RenderEngine } from "renderer";
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

    const engine = new RenderEngine(canvas, socket, toLobby);
    engine.init();
  }, [canvasRef, socket, toLobby]);

  return (
    <div className={styles.container}>
      <canvas ref={canvasRef} className={styles.canvas}></canvas>
    </div>
  );
}
