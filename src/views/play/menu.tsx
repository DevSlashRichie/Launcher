import styles from './menu.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
    faGear,
    faUser,
    faDiceD20,
    faCirclePlay,
} from '@fortawesome/free-solid-svg-icons';
import { Button } from '../../components/button/button';
import { OptionsBox } from '../../components/optionsbox/OptionsBox';
import { useOutsideClick } from '../../hooks/useOutsideClick';
import { AccountPicker } from './accounts/accountPicker';
import { invoke } from '@tauri-apps/api';
import { GamePicker } from './games/gamePicker';
import { useGames } from '../../stores/stores';

export function Menu() {
    const {
        isActive: showGames,
        el: gamesRef,
        btnRef: gamesBtnRef,
        setIsActive: setShowGames,
    } = useOutsideClick(false);

    const {
        isActive: showAccounts,
        el: accountsRef,
        btnRef: accountBtnRef,
    } = useOutsideClick(false);

    const { selectedGame } = useGames();

    return (
        <div className={styles.menu}>
            <div>
                <OptionsBox
                    className={styles.settingsBox}
                    show={showAccounts}
                    ref={accountsRef}
                >
                    {showAccounts && <AccountPicker />}
                </OptionsBox>
                <OptionsBox
                    className={styles.settingsBox}
                    show={showGames}
                    ref={gamesRef}
                >
                    <GamePicker />
                </OptionsBox>
            </div>
            <div className={styles.playCenter}>
                <div className={styles.playCenter__left}>
                    <Button
                        className={styles.play}
                        onClick={() => {
                            if (selectedGame) {
                                invoke('start_game').catch(console.error);
                            } else {
                                setShowGames(state => !state);
                            }
                        }}
                    >
                        <span>{selectedGame ? 'PLAY' : 'SELECT GAME'}</span>
                    </Button>
                    {selectedGame && (
                        <div className={styles.game}>
                            <FontAwesomeIcon icon={faCirclePlay} />
                            <span>The Box</span>
                        </div>
                    )}
                </div>

                <div className={styles.playCenter__right}>
                    <Button className={styles.sideBtn} ref={accountBtnRef}>
                        <FontAwesomeIcon icon={faUser} />
                    </Button>
                    <Button className={styles.sideBtn} ref={gamesBtnRef}>
                        <FontAwesomeIcon icon={faDiceD20} />
                    </Button>
                    <Button className={styles.sideBtn}>
                        <FontAwesomeIcon icon={faGear} />
                    </Button>
                </div>
            </div>
        </div>
    );
}
