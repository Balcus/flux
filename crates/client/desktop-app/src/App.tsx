import { Outlet } from "react-router-dom";
import Menu from "./components/Menu";
import { useRepository } from "./context/RepositoryContext";
import OpenRepository from "./components/OpenRepository";
import "./App.css";
import { ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";

function App() {
  const { repository } = useRepository();

  return (
    <>
      <ToastContainer
        position="bottom-right"
        autoClose={3000}
        newestOnTop
        closeOnClick
        rtl={false}
        pauseOnFocusLoss
        draggable
        pauseOnHover
        theme="light"
        hideProgressBar={true}
      />

      {!repository ? (
        <OpenRepository />
      ) : (
        <main className="app-layout">
          <Menu />
          <div className="content">
            <Outlet />
          </div>
        </main>
      )}
    </>
  );
}

export default App;
