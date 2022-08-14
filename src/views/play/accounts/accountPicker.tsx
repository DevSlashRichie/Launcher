
import {faArrowLeftLong, faCirclePlus} from "@fortawesome/free-solid-svg-icons";
import {useState} from "react";

import {Account} from "./account";
import styles from '../menu.module.scss';
import {Button} from "../../../components/button/button";

// tauri
import {invoke} from '@tauri-apps/api/tauri';
import {LoadingScreen} from "../loadingScreen";
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import {useArray} from "../../../hooks/useArray";

interface IAccount {
    name: string;
}

export function AccountPicker() {


    const [ loading, setLoading ] = useState(false);
    const {arr: accounts, addItem: addAccount, removeItem: removeAccount} = useArray<IAccount>();

    const handleClickAccountProvider = () => {
        setLoading(true);
        // invoke('auth_client').then();
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

            {loading && <LoadingScreen/>}
        </>
    )
}