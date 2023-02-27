import create from 'zustand';
import { invoke } from '@tauri-apps/api';
import { Account, AccountStore } from './account';
import { useEffect } from 'react';
import { Game, GameStore } from './game';

export const useGames = create<GameStore>((set) => ({
    games: [] as Array<Game>,

    fetchGames: async () => {
        const [games, electedGame] = await invoke<[Array<Game>, string | undefined]>('get_games');

        const selectedGame = electedGame ? games.find(g => g.id === electedGame) : undefined;

        set(() => ({
            games,
            selectedGame
        }));
    },

    pickGame: (id: string) => {
        set(state => {
            const game = state.games.find(game => game.id === id);

            if (game) {
                invoke('pick_game', {
                    id,
                }).then();

                return ({
                    ...state,
                    selectedGame: game,
                });
            } else
                return state;
        });
    }
}));

export const useAccounts = create<AccountStore>(set => ({
    accounts: [] as Array<Account>,

    addAccount: (account: Account) =>
        set(({ accounts, electedAccount }) => ({
            accounts: [...accounts, account],
            electedAccount,
        })),

    removeAccount: (index: number) => {
        invoke('remove_account', {
            account: index,
        });

        set(({ electedAccount, accounts }) => {
            const removeIsElected =
                electedAccount && accounts[index].uuid === electedAccount.uuid;

            return {
                accounts: accounts.filter((_, i) => i !== index),
                electedAccount: removeIsElected ? undefined : electedAccount,
            };
        });
    },

    setAccounts: (accounts: Array<Account>) =>
        set(({ electedAccount }) => ({ accounts, electedAccount })),

    fetchAccounts: async () => {
        const store = await invoke<any>('get_accounts');

        const electedAccount: Account | undefined = !store.elected_account
            ? undefined
            : [
                store.accounts.find(
                    (it: any) => it.profile.id === store.elected_account,
                ),
            ].map(
                (it: any) => ({
                    uuid: it.profile.id,
                    username: it.profile.name,
                } as Account),
            )[0];

        const accounts: Array<Account> = store.accounts.map(
            (it: any) => ({
                uuid: it.profile.id,
                username: it.profile.name,
            } as Account),
        );

        set(() => ({
            accounts,
            electedAccount,
        }));
    },

    pickAccount: async (index: number) => {
        await invoke('elect_account', {
            account: index,
        });

        set(state => ({
            ...state,
            electedAccount: state.accounts[index],
        }));
    },
}));

export function useInitializeStores() {
    const accounts = useAccounts();
    const games = useGames();
    useEffect(() => {
        accounts.fetchAccounts().then();
        games.fetchGames().then();
    }, []);
}
