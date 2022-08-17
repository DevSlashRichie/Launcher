import React, { useEffect, useState } from 'react';

export function useOutsideClick(
    el: React.RefObject<HTMLDivElement>,
    btnRef: React.RefObject<HTMLElement>,
    initialState: boolean,
    onOutsideClick?: () => void,
): boolean {

    const [ isActive, setIsActive ] = useState(initialState);

    useEffect(() => {
        const onClick = (ev: MouseEvent) => {
            if (btnRef.current) {
                if (btnRef.current.contains(ev.target as Node) || btnRef.current.isEqualNode(ev.target as Node)) {
                    setIsActive(old => {
                        if(old) {
                            onOutsideClick && onOutsideClick();
                        }
                        return !old;
                    });
                    return;
                }
            }

            if (el.current) {
                if (!el.current.contains(ev.target as Node)) {
                    setIsActive(false);
                    if(onOutsideClick) {
                        onOutsideClick();
                    }
                }
            }
        };

        window.addEventListener('click', onClick);

        return () => {
            window.removeEventListener('click', onClick);
        };

    }, [ isActive, el, btnRef ]);

    return isActive;
}
