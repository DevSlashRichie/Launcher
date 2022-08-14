import styles from './menu.module.scss';
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import {faGear, faUser, faDiceD20} from "@fortawesome/free-solid-svg-icons";
import {Button} from "../../components/button/button";
import {OptionsBox} from "../../components/optionsbox/OptionsBox";
import {useArray} from "../../hooks/useArray";
import {useRef, useState} from "react";
import {useOutsideClick} from "../../hooks/useOutsideClick";
import {AccountPicker} from "./accounts/accountPicker";


export function Menu() {

    const accountsRef = useRef<HTMLDivElement>(null);
    const accountBtnRef = useRef<HTMLDivElement>(null);
    const showAccounts = useOutsideClick(accountsRef, accountBtnRef, false);


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
                    { showAccounts && <AccountPicker /> }
                </OptionsBox>
            </div>
            <div className={styles.playCenter}>


                <Button className={styles.play}>
                    PLAY
                </Button>

                <div className={styles.playCenter__left}>
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
