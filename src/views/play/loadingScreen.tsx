import styles from './menu.module.scss';
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";
import {faSpinner, faCircleCheck} from "@fortawesome/free-solid-svg-icons";

export function LoadingScreen({message, done}: { message?: string, done?: boolean }) {
    return (
        <div className={styles.loadScreen}>

            {
                done
                    ? <div className={styles.loadScreen__done}>
                        <FontAwesomeIcon icon={faCircleCheck}/>
                        {message && <span>{message}</span>}
                    </div>
                    : <div>
                        <FontAwesomeIcon icon={faSpinner} spin/>
                        <span>{message || 'Authenticating'} </span>
                    </div>
            }
        </div>
    );

}