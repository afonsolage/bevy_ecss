# Theme example

<script type="module">
    // Import and run your bevy wasm code
    import init from './theme.js'
    init();
    document.addEventListener('readystatechange', event => { 
        if (event.target.readyState === "complete") {
            setTimeout(function() {
                var canvas = document.getElementsByTagName('canvas');
                var main = document.getElementsByTagName('main');
                for(var i = 0; i < canvas.length; i++) {
                    main[0].appendChild(canvas[i])
                    canvas[i].setAttribute('style','width:100%;');
                }
            }, 1000);
        }
    });
</script>