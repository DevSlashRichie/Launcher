
import {faArrowLeftLong, faCirclePlus} from "@fortawesome/free-solid-svg-icons";
import {useState} from "react";

import {Account} from "./account";
import styles from '../menu.module.scss';
import {Button} from "../../../components/button/button";

// tauri
import {invoke} from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';

import {LoadingScreen} from "../loadingScreen";
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import {useArray} from "../../../hooks/useArray";

interface IAccount {
    name: string;
}

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
    const {arr: accounts, addItem: addAccount, removeItem: removeAccount} = useArray<IAccount>();

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
                        name: message
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
                            name={it.name}
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