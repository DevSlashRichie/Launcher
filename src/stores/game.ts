

export type Game = {
    id: string,
    name: string,
};

export type GameStore = {
    games: Array<Game>,

    selectedGame?: Game,
    fetchGames: () => Promise<void>,
    pickGame: (game: string) => void,
};