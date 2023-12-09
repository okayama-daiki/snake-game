import { useState } from "react";
import Lobby from "./components/Lobby";

export default function App() {
  const [isGameStarted, setIsGameStarted] = useState(false);
  const toGame = () => setIsGameStarted(true);

  return <main>{isGameStarted ? <></> : <Lobby toGame={toGame} />}</main>;
}
