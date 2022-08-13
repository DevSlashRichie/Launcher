import styles from './optionsbox.module.scss';
import {forwardRef, ReactNode} from "react";
import cn from 'classnames';

interface OptionsBoxProps {
    children: ReactNode,
    className?: string,
    show?: boolean
}

export const OptionsBox = forwardRef<HTMLDivElement, OptionsBoxProps>(({children, className, show}, ref) => {
    return (
        <div>
            {
                show &&
                <div className={cn(className, styles.optionsbox)} ref={ref}>
                    {children}
                </div>
            }
        </div>

    );
});

OptionsBox.displayName = 'OptionsBox';