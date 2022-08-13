import {forwardRef, ReactNode} from "react";
import styles from './button.module.scss';
import cn from 'classnames';

interface BtnProps {
    children: ReactNode,
    className?: string,
    onClick?: () => void
}

export const Button = forwardRef<HTMLDivElement, BtnProps>(({onClick, className, children}, ref) => {
    const handleClick = () => {
        if (onClick) {
            onClick();
        }
    }

    return <div className={cn(className, styles.btn)} onClick={handleClick} role='button' ref={ref}>
        {children}
    </div>;
});

Button.displayName = 'Button';