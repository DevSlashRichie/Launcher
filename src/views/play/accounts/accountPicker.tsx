import { useState } from 'react';

import { faCirclePlus } from '@fortawesome/free-solid-svg-icons';

import { Account } from './account';
import { useAccounts } from '../../../stores/stores';
import styles from '../menu.module.scss';

// tauri
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

import { LoadingScreen } from '../loadingScreen';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

type LoadState = {
    state: 'LOADING' | 'DONE' | 'ERROR';
    message?: string;
};

interface AuthStateEvent {
    state: 'INFO' | 'DONE' | 'ERROR';
    message: string;
}

export function AccountPicker() {
    const [loading, setLoading] = useState<LoadState | null>(null);

    const { accounts, fetchAccounts, removeAccount, pickAccount } = useAccounts();

    const handleClickAccountProvider = () => {
        setLoading({ state: 'LOADING' });

        let off: null | (() => void) = null;
        listen<AuthStateEvent>(
            'auth:state',
            ({ payload: { state, message } }) => {
                if (state === 'INFO') {
                    setLoading({ state: 'LOADING', message });
                } else {
                    if (state === 'DONE') {
                        fetchAccounts().then(() => {
                            setLoading({
                                state: 'DONE',
                                message: `Logged in as ${message}`,
                            });
                        });
                    } else if (state === 'ERROR') {
                        setLoading({
                            state: 'ERROR',
                            message: message,
                        });
                    }

                    if (off) off();
                    setTimeout(() => {
                        setLoading(null);
                    }, 1500);
                }
            },
        )
            .then(_off => {
                off = _off;
                invoke('add_account').then();
            })
            .catch(console.error);
    };

    return (
        <>
            <div
                className={styles.addAccount}
                onClick={() => handleClickAccountProvider()}
            >
                <span>
                    <FontAwesomeIcon icon={faCirclePlus} />
                </span>
            </div>
            <div>
                {accounts.map((it, index) => (
                    <Account
                        key={index}
                        name={it.username}
                        onRemove={() => removeAccount(index)}
                        onPick={() => pickAccount(index)}
                    />
                ))}
            </div>

            {loading && (
                <LoadingScreen
                    state={loading.state}
                    message={loading.message}
                />
            )}
        </>
    );
}
