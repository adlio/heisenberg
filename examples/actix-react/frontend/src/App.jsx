import { BrowserRouter, Routes, Route, Link, useLocation } from 'react-router-dom'
import Home from './pages/Home'
import About from './pages/About'
import ApiDemo from './pages/ApiDemo'
import './App.css'

function Navigation() {
  const location = useLocation()
  
  return (
    <nav>
      <Link to="/" className={location.pathname === '/' ? 'active' : ''}>
        Home
      </Link>
      <Link to="/about" className={location.pathname === '/about' ? 'active' : ''}>
        About
      </Link>
      <Link to="/api-demo" className={location.pathname === '/api-demo' ? 'active' : ''}>
        API Demo
      </Link>
    </nav>
  )
}

function App() {
  return (
    <BrowserRouter>
      <Navigation />
      <div className="container">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/about" element={<About />} />
          <Route path="/api-demo" element={<ApiDemo />} />
        </Routes>
      </div>
    </BrowserRouter>
  )
}

export default App
