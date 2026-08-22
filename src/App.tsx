import { useCallback, useEffect, useState } from "react";
import ErrorModal from "./components/Error";
import Game from "./components/Game";
import Lobby from "./components/Lobby";
import init from "./services/renderer/pkg";
import { ConnectionStatus, PlayerStatus, type RankingEntry } from "./types";

const websocketUri = import.meta.env.VITE_WSS_URI || "ws://localhost:5173";
const socket = new WebSocket(websocketUri);
const playerToken = crypto.randomUUID();

const leaderboardUri = (() => {
  if (import.meta.env.VITE_HTTP_URI) {
    return new URL("/leaderboard", import.meta.env.VITE_HTTP_URI).toString();
  }
  const url = new URL(websocketUri);
  url.protocol = url.protocol === "wss:" ? "https:" : "http:";
  url.pathname = "/leaderboard";
  url.search = "";
  url.hash = "";
  return url.toString();
})();

init();

export default function App() {
  const [playerStatus, setPlayerStatus] = useState(PlayerStatus.NOT_PLAYING);
  const [connectionStatus, setConnectionStatus] = useState(
    ConnectionStatus.CONNECTING,
  );
  const [ranking, setRanking] = useState<RankingEntry[]>([]);
  const toLobby = useCallback(() => {
    setRanking([]);
    setPlayerStatus(PlayerStatus.NOT_PLAYING);
  }, []);
  const toGame = useCallback(() => {
    setRanking([]);
    setPlayerStatus(PlayerStatus.PLAYING);
  }, []);

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

  useEffect(() => {
    if (playerStatus !== PlayerStatus.PLAYING) return;

    let isMounted = true;
    const refreshRanking = async () => {
      try {
        const url = new URL(leaderboardUri);
        url.searchParams.set("player", playerToken);
        const response = await fetch(url);
        if (!response.ok) return;
        const entries: RankingEntry[] = await response.json();
        if (isMounted) setRanking(entries);
      } catch {
        // The connection status UI already handles an unavailable backend.
      }
    };

    refreshRanking();
    const intervalId = window.setInterval(refreshRanking, 2_000);
    return () => {
      isMounted = false;
      window.clearInterval(intervalId);
    };
  }, [playerStatus]);

  return (
    <main>
      {connectionStatus === ConnectionStatus.CLOSED && (
        <ErrorModal transparent={playerStatus === PlayerStatus.PLAYING} />
      )}
      {playerStatus === PlayerStatus.PLAYING && (
        <Game
          socket={socket}
          toLobby={toLobby}
          ranking={ranking}
          playerToken={playerToken}
        />
      )}
      {playerStatus === PlayerStatus.NOT_PLAYING && (
        <Lobby connectionStatus={connectionStatus} toGame={toGame} />
      )}
    </main>
  );
}
