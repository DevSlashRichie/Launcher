import {Dispatch, SetStateAction, useEffect, useRef, useState} from 'react';

export function useOutsideClick(
    initialState: boolean,
    onOutsideClick?: () => void,
) {

    const [ isActive, setIsActive ] = useState(initialState);

    const el = useRef<HTMLDivElement>(null);
    const btnRef = useRef<HTMLDivElement>(null);

    const [stateBlocker, setStateBlocker] = useState(false);
    const handleActive = (state: SetStateAction<boolean>) => {
        setStateBlocker(true);
        setIsActive(state);
        setTimeout(() => {
            setStateBlocker(false);
        }, 10);
    };

    useEffect(() => {
        if (stateBlocker)
            return;
        
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

    }, [ isActive, el, btnRef, stateBlocker ]);

    return { isActive, el, btnRef, setIsActive: handleActive };
}
