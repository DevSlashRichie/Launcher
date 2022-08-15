import {Menu} from "./menu";
import styles from './play.module.scss';
import bg from '../../assets/LOGO.svg';

export function Play() {
    return (
        <div className={styles.play}>
            <div className={ styles.play__container }>
                <div className={ styles.upperMenu }>
                    <div>
                        Minecraft Launcher: Cognatize Edition
                    </div>
                    <div>
                        <img src="https://crafatar.com/renders/head/55746d4b26f94f3380c1763b63efa66c" alt=""/>
                        <span>ZetaRicardo</span>
                    </div>
                </div>
                <div className={styles.logo}>
                    <div className={styles.center}>
                        <img src={bg} alt="Logo"/>
                        <div className={styles.center__text}>Are u ready to play?</div>
                    </div>
                </div>
                <Menu/>
            </div>

            <div className={ styles.overflow }>
                Lorem ipsum dolor sit amet, consectetur adipisicing elit. Autem consequuntur non placeat quaerat? A ab
                aspernatur doloremque ea earum, inventore, ipsum laborum nihil, quae quibusdam quis sint sunt totam
                veniam!
            </div>
        </div>
    )
}