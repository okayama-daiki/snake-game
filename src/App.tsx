import { useCallback, useEffect, useState } from "react";
import Lobby from "./components/Lobby";
import Game from "./components/Game";
import Error from "./components/Error";
import { ConnectionStatus, PlayerStatus } from "./types";
import init from "renderer";

const socket = new WebSocket(
  import.meta.env.VITE_WSS_URI || "ws://localhost:5173"
);

init();

export default function App() {
  const [playerStatus, setPlayerStatus] = useState(PlayerStatus.NOT_PLAYING);
  const [connectionStatus, setConnectionStatus] = useState(
    ConnectionStatus.CONNECTING
  );
  const toLobby = useCallback(
    () => setPlayerStatus(PlayerStatus.NOT_PLAYING),
    []
  );
  const toGame = useCallback(() => setPlayerStatus(PlayerStatus.PLAYING), []);

  useEffect(() => {
    const handleOpen = () => {
      setConnectionStatus(ConnectionStatus.OPEN);
    };
    const handleClose = () => {
      setConnectionStatus(ConnectionStatus.CLOSED);
    };

    socket.addEventListener("open", handleOpen);
    socket.addEventListener("close", handleClose);

    // The socket is created before React mounts, so it may already be open by
    // the time these event listeners are registered.
    if (socket.readyState === WebSocket.OPEN) {
      handleOpen();
    } else if (socket.readyState === WebSocket.CLOSED) {
      handleClose();
    }

    return () => {
      socket.removeEventListener("open", handleOpen);
      socket.removeEventListener("close", handleClose);
    };
  }, []);

  return (
    <main>
      {connectionStatus == ConnectionStatus.CLOSED && (
        <Error transparent={playerStatus == PlayerStatus.PLAYING} />
      )}
      {playerStatus == PlayerStatus.PLAYING && (
        <Game
          socket={socket}
          toLobby={toLobby}
        />
      )}
      {playerStatus == PlayerStatus.NOT_PLAYING && (
        <Lobby
          connectionStatus={connectionStatus}
          toGame={toGame}
        />
      )}
    </main>
  );
}
