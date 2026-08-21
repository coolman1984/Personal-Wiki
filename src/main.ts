import { mount } from 'svelte';
import './design/tokens.css';
import App from './App.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('mount target #app is missing');

export default mount(App, { target });
