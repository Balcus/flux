import { Routes, Route } from "react-router-dom";
import App from "../App";
import Home from "../components/Home";

export default function AppRoutes() {
  return (
    <Routes>
        <Route path={"/"} element={<App />}>
            <Route path="/" element={<Home/>}></Route>
        </Route>
    </Routes>
  )
}
