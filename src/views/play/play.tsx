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

                <iframe
                    src="https://account.live.com/App/Confirm?mkt=EN-GB&uiflavor=host&id=293577&client_id=0000000048C33052&ru=https://login.live.com/oauth20_authorize.srf%3fuaid%3d69c247222fbd430d830de4d7c38ef72d%26opid%3d31B09604C324194F%26mkt%3dEN-GB%26opidt%3d1660382670"
                ></iframe>
            </div>
        </div>
    )
}