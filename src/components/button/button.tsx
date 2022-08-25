import { forwardRef, ReactNode } from 'react';
import styles from './button.module.scss';
import cn from 'classnames';

interface BtnProps {
    children: ReactNode;
    className?: string;
    onClick?: () => void;
    disabled?: boolean;
}

export const Button = forwardRef<HTMLDivElement, BtnProps>(
    ({ onClick, className, children, disabled }, ref) => {
        const handleClick = () => {
            if (onClick) {
                onClick();
            }
        };

        return (
            <div
                className={cn(className, styles.btn, {
                    [styles.disabled]: disabled,
                })}
                onClick={handleClick}
                role="button"
                ref={ref}
            >
                {children}
            </div>
        );
    },
);

Button.displayName = 'Button';
