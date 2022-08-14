import styles from './menu.module.scss';
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import {faSpinner} from "@fortawesome/free-solid-svg-icons";

export function LoadingScreen() {
    return (
        <div className={styles.loadScreen}>
            <FontAwesomeIcon icon={faSpinner} spin/>
            <span>Authenticating</span>
        </div>
    );

}