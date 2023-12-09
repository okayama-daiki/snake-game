import { useEffect, useRef } from "react";

function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) {
      return;
    }
    const canvas = canvasRef.current;
    const context = canvas.getContext("2d");

    context;
  }, [canvasRef]);

  return (
    <main>
      <canvas ref={canvasRef} />
    </main>
  );
}

export default App;
