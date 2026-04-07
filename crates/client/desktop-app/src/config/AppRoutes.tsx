import { Routes, Route } from "react-router-dom";
import App from "../App";
import Home from "../components/Home";
import Settings from "../components/Settings";
import History from "../components/History";
import FileStatus from "../components/FileStatus";

export default function AppRoutes() {
  return (
    <Routes>
        <Route path={"/"} element={<App />}>
            <Route path="/" element={<Home/>}></Route>
            <Route path="/settings" element={<Settings/>}></Route>
            <Route path="/history" element={<History/>}></Route>
            <Route path="file-status" element={<FileStatus />}></Route>
        </Route>
    </Routes>
  )
}
