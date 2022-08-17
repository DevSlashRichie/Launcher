export type Account = {
    username: string;
    uuid: string;
};

export type AccountStore = {
    accounts: Array<Account>;
    electedAccount?: Account;

    addAccount: (account: Account) => void;
    removeAccount: (index: number) => void;
    setAccounts: (accounts: Array<Account>) => void;
    fetchAccounts: () => Promise<void>;
    pickAccount: (index: number) => Promise<void>;
};
