import { Outlet } from 'react-router-dom';
import Menu from './components/Menu'
import { useRepository } from './context/RepositoryContext';
import OpenRepository from './components/OpenRepository';

import "./App.css";

function App() {
  const { repository } = useRepository();

  if (!repository) {
    return <OpenRepository />;
  }

  return (
    <main>
      <Menu />
      <Outlet />
    </main>
  );
}

export default App;
