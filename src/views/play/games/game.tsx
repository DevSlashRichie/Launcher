import styled from 'styled-components';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faPlayCircle } from '@fortawesome/free-solid-svg-icons';

interface MenuProps {
    name: string;
    picked?: boolean;
    onClick: () => void;
}

const Container = styled.div<MenuProps>`
    padding: 20px;
    gap: 20px;
    font-size: 18px;

    display: flex;
    justify-content: space-between;

    background: ${props => (props.picked ? '' : 'rgba(255, 255, 255, 0.1)')};
    cursor: pointer;

    &:hover {
        background: rgba(255, 255, 255, 0.1);
    }

    & *:hover {
        cursor: pointer;
    }

    & > div {
        line-height: -2;
    }
`;

const PlayIcon = styled.div`
    font-size: 20px;
`;

export function Game({ name, picked, onClick }: MenuProps) {
    return (
        <Container {...{ name, picked, onClick }}>
            <span>{name}</span>
            <PlayIcon>
                <FontAwesomeIcon icon={faPlayCircle} />
            </PlayIcon>
        </Container>
    );
}
