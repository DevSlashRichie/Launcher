import {useState} from "react";


export function useArray<T>(initialValue: T[] | (() => T[]) = []) {

    const [ arr, setArr ] = useState<Array<T>>(initialValue);

    const addItem = (item: T) => {
        setArr([...arr, item]);
    }

    const removeItem = (index: number) => {
        setArr(arr.filter((_, i) => i !== index));
    }

    const setItem = (index: number, item: T) => {
        const newArr = [...arr];
        newArr[index] = item;
        setArr(newArr);
    }

    const setItems = (items: T[]) => {
        setArr(items);
    }

    return { arr, addItem, removeItem, setItems, setItem };
}