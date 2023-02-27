import { useAccounts, useGames } from '../../stores/stores';
import { PlayBox } from './play-box';

import { useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { useOutsideClick } from '../../hooks/useOutsideClick';

interface AuthStateEvent {
    state: 'INFO' | 'DONE' | 'ERROR';
    message: string;
}

export function Play() {
    const { electedAccount, removeAccount, accounts, fetchAccounts } = useAccounts();
    const { selectedGame } = useGames();

    const {
        isActive: menuIsActive,
        el: menuRef,
        btnRef: menuBtnRef,
        setIsActive: setMenuIsActive,
    } = useOutsideClick(false);

    const [notLoggedInMsg, setNotLoggedInMsg] = useState('Not logged in!');

    const handleToggleMenu = () => {
        if (electedAccount)
            setMenuIsActive(!menuIsActive);
        else {
            setMenuIsActive(false);
            setNotLoggedInMsg('Loading...');
            let off: null | (() => void) = null;
            listen<AuthStateEvent>(
                'auth:state',
                ({ payload: { state, message } }) => {
                    if (state === 'INFO') {
                        setNotLoggedInMsg(message);
                    } else {
                        if (state === 'DONE') {
                            fetchAccounts().then(() => {
                                setNotLoggedInMsg('Logged in as ${message}');
                            });
                        } else if (state === 'ERROR') {
                            setNotLoggedInMsg('Error: ${message}');
                        }

                        if (off) off();
                        setTimeout(() => {
                            setNotLoggedInMsg('');
                        }, 1500);
                    }
                }
            )
                .then(_off => {
                    off = _off;
                    invoke('add_account').then();
                })
                .catch(console.error);
        }
    };


    return (
        <div className='bg-black w-full h-screen text-white'>
            <div className="w-full h-16 flex justify-end p-3">
                {
                    menuIsActive
                    && <div
                        className='w-32 bg-[#1d1d1d] flex flex-col p-3 absolute uppercase text-sm mt-[45px] gap-3 rounded'
                        ref={menuRef}
                    >
                        <button className='text-lg'>
                            Settings
                        </button>
                        <button
                            className='text-lg'
                            onClick={() => {
                                const index = accounts.findIndex(account => account.uuid === electedAccount?.uuid);
                                removeAccount(index);
                                setMenuIsActive(false);
                                setNotLoggedInMsg('Not logged in!');
                            }}
                        >
                            Exit
                        </button>
                    </div>

                }
                <div onClick={handleToggleMenu} ref={menuBtnRef}>
                    {
                        electedAccount
                            ? <>
                                <button
                                    className='flex gap-2 border rounded py-2 px-4 cursor-pointer transition-all'
                                >
                                    <img
                                        width={25}
                                        height={25}
                                        src={`https://crafatar.com/renders/head/${electedAccount.uuid}`}
                                    />
                                    <span>
                                        {electedAccount.username}
                                    </span>
                                </button>
                            </>
                            : <>
                                <button className='border rounded py-2 px-4'>{notLoggedInMsg}</button>
                            </>
                    }
                </div>
            </div>
            <div className='w-full h-[300px] bg-fixed bg-center bg-no-repeat bg-cover p-14 bg-[url("/src/assets/splash.jpg")] flex flex-col items-center justify-center'>
                <button
                    disabled={!selectedGame || !electedAccount}
                    className='relative bg-white text-black text-4xl rounded-xl cursor-pointer border-b-8 border-b-[gray] active:translate-y-[3px] active:scale-[97%] transition-all hover:bg-white/90 w-[75%] disabled:bg-[#BBBBBB]'
                    onClick={() => {
                        invoke('start_game').catch(console.error);
                    }}
                >
                    <div className='absolute w-[0%] h-full z-0 bg-black/20' />
                    <div className='flex flex-col text-center justify-center items-center p-2'>
                        Launch Game
                        {<span className='text-sm'>{selectedGame?.name || 'pick a game'}</span>}
                    </div>
                </button>
            </div>
            <div className='w-full flex flex-wrap justify-center items-center gap-5 p-8 bg-black'>
                <PlayBox />
                <PlayBox />
                <PlayBox />
                <PlayBox />
            </div>
        </div >
    );
}
