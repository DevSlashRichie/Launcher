import { Game } from './game';
import { useGames } from '../../../stores/stores';

export function GamePicker() {
    const { games, pickGame, selectedGame } = useGames();

    return (
        <div>
            {games.map(({ name, id }) => (
                <Game
                    key={id}
                    name={name}
                    picked={selectedGame?.id === id}
                    onClick={() => pickGame(id)}
                />
            ))}
        </div>
    );
}
