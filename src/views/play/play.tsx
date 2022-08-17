import { Menu } from './menu';
import styles from './play.module.scss';
import bg from '../../assets/LOGO.svg';
import { useAccounts } from '../../stores/stores';

export function Play() {
    const { electedAccount } = useAccounts();

    return (
        <div className={styles.play}>
            <div className={styles.play__container}>
                <div className={styles.upperMenu}>
                    <div>Minecraft Launcher: Cognatize Edition</div>
                    {electedAccount && (
                        <div>
                            <img
                                src={`https://crafatar.com/renders/head/${electedAccount.uuid}`}
                                alt=""
                            />
                            <span>{electedAccount.username}</span>
                        </div>
                    )}
                </div>
                <div className={styles.logo}>
                    <div className={styles.center}>
                        <img src={bg} alt="Logo" />
                        <div className={styles.center__text}>
                            Are u ready to play?
                        </div>
                    </div>
                </div>
                <Menu />
            </div>

            <div className={styles.overflow}>
                Lorem ipsum dolor sit amet, consectetur adipisicing elit. Autem
                consequuntur non placeat quaerat? A ab aspernatur doloremque ea
                earum, inventore, ipsum laborum nihil, quae quibusdam quis sint
                sunt totam veniam!
            </div>
        </div>
    );
}
