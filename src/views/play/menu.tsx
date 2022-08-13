import styles from './menu.module.scss';
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import {faGear, faUser, faArrowLeftLong, faCirclePlus, faDiceD20} from "@fortawesome/free-solid-svg-icons";
import {Button} from "../../components/button/button";
import {OptionsBox} from "../../components/optionsbox/OptionsBox";
import {useArray} from "../../hooks/useArray";
import {Account} from "./account";
import {useRef, useState} from "react";
import {useOutsideClick} from "../../hooks/useOutsideClick";

// tauri
import { invoke } from '@tauri-apps/api/tauri';

interface IAccount {
    name: string;
}

export function Menu() {

    const {arr: accounts, addItem: addAccount, removeItem: removeAccount} = useArray<IAccount>();

    const accountsRef = useRef<HTMLDivElement>(null);
    const accountBtnRef = useRef<HTMLDivElement>(null);
    const [showAddAccount, setShowAddAccount] = useState(false);
    const showAccounts = useOutsideClick(accountsRef, accountBtnRef, false, () => {
        setShowAddAccount(false);
    });

    const handleClickAccountProvider = (msc: boolean) => {
        invoke('auth_client').then();
    }


    return (
        <div
            className={styles.menu}
        >
            <div>
                <OptionsBox
                    className={styles.settingsBox}
                    show={showAccounts}
                    ref={accountsRef}
                >
                    {
                        showAddAccount
                        && <div className={styles.addAccountView}>
                            <div className={styles.addAccountView__addAcc}>
                                Agregar cuenta
                            </div>
                            <div className={styles.addAccountView__opts}>
                                <div>
                                    <Button
                                        className={styles.addAccountView__opts_btn}
                                        onClick={() => handleClickAccountProvider(true)}
                                    >
                                        MICROSOFT
                                    </Button>
                                </div>
                                <div>
                                    <Button className={styles.addAccountView__opts_btn}>
                                        MOJANG
                                    </Button>
                                </div>
                            </div>
                            <div
                                className={styles.addAccountView__close}
                                onClick={ev => {
                                    ev.stopPropagation();
                                    setShowAddAccount(false)
                                }}
                            >
                                <FontAwesomeIcon icon={faArrowLeftLong}/>
                            </div>
                        </div>
                    }

                    <div
                        className={styles.addAccount}
                        onClick={() => setShowAddAccount(!showAddAccount)}
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
                </OptionsBox>
            </div>
            <div className={styles.playCenter}>


                <Button className={styles.play}>
                    PLAY
                </Button>

                <div className={ styles.playCenter__left }>
                    <Button className={styles.sideBtn} ref={accountBtnRef}>
                        <FontAwesomeIcon icon={faUser}/>
                    </Button>
                    <Button className={styles.sideBtn}>
                        <FontAwesomeIcon icon={faDiceD20}/>
                    </Button>
                    <Button className={styles.sideBtn}>
                        <FontAwesomeIcon icon={faGear}/>
                    </Button>
                </div>

            </div>

        </div>
    );
}
