import styles from '../menu.module.scss';
import { faCircleMinus } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import cn from 'classnames';

export function Account({
    name,
    onRemove,
    picked,
    onPick
}: {
    name: string;
    onRemove: () => void;
    onPick: () => void;
    picked?: boolean;
}) {
    return (
        <div
            className={cn(styles.account, {
                [styles.picked]: picked,
            })}
            onClick={onPick}
        >
            <span>{name}</span>
            <span
                onClick={(ev) => {
                    ev.stopPropagation();
                    onRemove();
                }}
            >
                <FontAwesomeIcon icon={faCircleMinus} />
            </span>
        </div>
    );
}
