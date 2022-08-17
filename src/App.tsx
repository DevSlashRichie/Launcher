import { Play } from './views/play/play';
import { useInitializeStores } from './stores/stores';

function App() {
    useInitializeStores();

    return <Play />;
}

export default App;
