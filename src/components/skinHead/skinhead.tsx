import { useEffect, useState } from 'react';

export function SkinHead({ name }: { name: string }) {

    const [ image, setImage ] = useState<string>();

    useEffect(() => {
        fetch(`https://minecraft-api.com/api/skins/${name}/head/-10.-25/0/5/json`)
            .then(res => {
                res.json()
                    .then(data => {
                        console.log('adasdas');
                        setImage(`data:image/png;base64,\n${data.head}`);
                    })
                    .catch(console.error);
            })
            .catch(console.error);
    }, [ name ]);

    return <img src={image} alt={name}/>;
}