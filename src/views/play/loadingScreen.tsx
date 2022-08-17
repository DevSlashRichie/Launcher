import styles from './menu.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faSpinner, faCircleCheck, faBomb } from '@fortawesome/free-solid-svg-icons';

export function LoadingScreen({
    message,
    state,
}: {
    message?: string;
    state?: 'LOADING' | 'DONE' | 'ERROR';
}) {
    const retrieve = () => {
        switch (state) {
            case 'LOADING':
                return (
                    <div>
                        <FontAwesomeIcon icon={faSpinner} spin />
                        <span>{message || 'Authenticating'} </span>
                    </div>
                );
            case 'DONE':
                return (
                    <div className={styles.loadScreen__done}>
                        <FontAwesomeIcon icon={faCircleCheck} />
                        {message && <span>{message}</span>}
                    </div>
                );
            case 'ERROR':
                return (
                    <div className={styles.loadScreen__error}>
                        <FontAwesomeIcon icon={faBomb} />
                        {message && <span>{message}</span>}
                    </div>
                );
        }
    };

    return <div className={styles.loadScreen}>{state && retrieve()}</div>;
}
