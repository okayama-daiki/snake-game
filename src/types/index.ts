enum ConnectionStatus {
  CONNECTING,
  OPEN,
  CLOSED,
}

enum PlayerStatus {
  NOT_PLAYING,
  PLAYING,
}

interface RankingEntry {
  name: string;
  score: number;
  is_bot: boolean;
  rank: number;
  is_self: boolean;
}

export type { RankingEntry };
export { ConnectionStatus, PlayerStatus };
