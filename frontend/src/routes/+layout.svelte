<script lang="ts">
  import "../app.css";
</script>

<div class="container">
  <nav>
    <ul>
      <li><a href="/campaigns">Campaigns</a></li>
      <li><a href="/etc">...</a></li>
    </ul>
  </nav>

  <main>
    <slot />
  </main>
</div>

<style>
  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }

  nav {
    background-color: #f4f4f4;
    padding: 1rem;
    margin-bottom: 2rem;
    border-radius: 8px;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    gap: 2rem;
  }

  a {
    text-decoration: none;
    color: #333;
    font-weight: bold;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  a:hover {
    background-color: #e0e0e0;
  }
</style> 