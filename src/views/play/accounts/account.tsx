import styles from '../menu.module.scss';
import {faCircleMinus} from "@fortawesome/free-solid-svg-icons";
import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";

export function Account({ name, onRemove }: { name: string, onRemove: () => void }) {
    return <div className={styles.account}>
        <span>{ name }</span>
        <span
            onClick={ ev => {
                ev.stopPropagation();
                onRemove();
            }}
        >
            <FontAwesomeIcon icon={faCircleMinus}/>
        </span>
    </div>
}