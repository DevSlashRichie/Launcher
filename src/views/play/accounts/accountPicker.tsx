
import {useEffect, useState} from "react";

import { faCirclePlus} from "@fortawesome/free-solid-svg-icons";
import create from 'zustand/vanilla';

import {Account} from "./account";
import styles from '../menu.module.scss';

// tauri
import {invoke} from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

import {LoadingScreen} from "../loadingScreen";
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import {useArray} from "../../../hooks/useArray";

interface IAccount {
    username: string;
}

const accountsStore = create<Array<IAccount>>(() => []);

function fetchAccounts() {
    invoke('get_accounts').then((accounts) => {
        accountsStore.setState(accounts as Array<IAccount>, true);
        console.log(accounts);
    });
}

fetchAccounts();

type LoadState = {
    loading: boolean,
    message?: string,
};

interface AuthStateEvent {
    state: 'INFO' | 'DONE',
    message: string,
}

export function AccountPicker() {


    const [ loading, setLoading ] = useState<LoadState | null>(null);
    const {arr: accounts, addItem: addAccount, removeItem: removeAccount, setItems: setAccounts} = useArray<IAccount>(accountsStore.getState);

    useEffect(() => accountsStore.subscribe(setAccounts), []);

    const handleClickAccountProvider = () => {
        setLoading({ loading: true });

        let off: null | (() => void)  = null;
        listen<AuthStateEvent>('auth:state', ({ payload: { state, message } }) => {
            if (state === 'INFO') {
                setLoading({ loading: true, message });
            } else if(state === 'DONE') {
                if(off) off();
                setLoading({ loading: false, message: `Logged in as ${message}` });
                setTimeout(() => {
                    addAccount({
                        username: message,
                    });
                    setLoading(null);
                }, 1500);
            }
        })
            .then(_off => {
                off = _off;
                invoke('auth_client').then();
            })
            .catch(console.error);
    }

    return (
        <>
            <div
                className={styles.addAccount}
                onClick={() => handleClickAccountProvider()}
            >
                <span>
                    <FontAwesomeIcon icon={faCirclePlus}/>
                </span>
            </div>
            <div>
                {
                    accounts.map((it, index) =>
                        <Account
                            key={index}
                            name={it.username}
                            onRemove={() => removeAccount(index)}
                        />
                    )
                }

            </div>

            {loading && <LoadingScreen
                done={ !loading.loading }
                message={ loading.message }
            />}
        </>
    )
}