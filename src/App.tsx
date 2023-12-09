import { useState } from "react";
import Lobby from "./components/Lobby";
import Game from "./components/Game";

export default function App() {
  const [isGameStarted, setIsGameStarted] = useState(false);
  const toGame = () => setIsGameStarted(true);
  const toLobby = () => setIsGameStarted(false);

  return (
    <main>
      {isGameStarted ? <Game toLobby={toLobby} /> : <Lobby toGame={toGame} />}
    </main>
  );
}
