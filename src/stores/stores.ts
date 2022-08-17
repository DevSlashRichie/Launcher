import create from 'zustand';
import { invoke } from '@tauri-apps/api';
import { Account, AccountStore } from './account';
import { useEffect } from 'react';

export const useAccounts = create<AccountStore>(set => ({
    accounts: [] as Array<Account>,

    addAccount: (account: Account) =>
        set(({ accounts, electedAccount }) => ({
            accounts: [...accounts, account],
            electedAccount,
        })),

    removeAccount: async (index: number) => {
        await invoke('remove_account', {
            account: index,
        });

        set(({ electedAccount, accounts }) => {
            const removeIsElected =
                electedAccount && accounts[index] === electedAccount;

            return {
                accounts: accounts.filter((_, i) => i !== index),
                electedAccount: removeIsElected ? undefined : electedAccount,
            };
        });
    },

    setAccounts: (accounts: Array<Account>) =>
        set(({ electedAccount }) => ({ accounts, electedAccount })),

    fetchAccounts: async () => {
        const store = await invoke<{
            accounts: Array<Account>;
            elected_account?: string;
        }>('get_accounts');

        const electedAccount = !store.elected_account
            ? undefined
            : store.accounts.find(it => it.uuid === store.elected_account);

        set(() => ({
            accounts: store.accounts,
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
    useEffect(() => {
        accounts.fetchAccounts().then();
    }, []);
}
