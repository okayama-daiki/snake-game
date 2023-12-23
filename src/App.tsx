import { useEffect, useState } from "react";
import Lobby from "./components/Lobby";
import Game from "./components/Game";
import ErrorModal from "./components/Modal/Error";
import { ConnectionStatus, PlayerStatus } from "./types";

const socket = new WebSocket(import.meta.env.VITE_WSS_URI);

export default function App() {
  const [playerStatus, setPlayerStatus] = useState(PlayerStatus.NOT_PLAYING);
  const [connectionStatus, setConnectionStatus] = useState(
    ConnectionStatus.CONNECTING
  );

  useEffect(() => {
    socket.onopen = () => {
      setConnectionStatus(ConnectionStatus.OPEN);
    };
    socket.onclose = () => {
      setConnectionStatus(ConnectionStatus.CLOSED);
    };
  });

  return (
    <main>
      {connectionStatus == ConnectionStatus.CLOSED && <ErrorModal />}
      {playerStatus == PlayerStatus.PLAYING && (
        <Game
          socket={socket}
          toLobby={() => setPlayerStatus(PlayerStatus.NOT_PLAYING)}
        />
      )}
      {playerStatus == PlayerStatus.NOT_PLAYING && (
        <>
          <Lobby
            connectionStatus={connectionStatus}
            toGame={() => setPlayerStatus(PlayerStatus.PLAYING)}
          />
        </>
      )}
    </main>
  );
}
