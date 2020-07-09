import loadImage from 'image-promise';

const rust = import('./pkg');

const f = async (visualObjects) => {
    console.log(visualObjects);

    const canvas = document.getElementById('display');
    const ctx = canvas.getContext('2d');

    console.log(ctx.width);
    let width = 600;
    let height = 400;
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    ctx.save();
    ctx.translate(canvas.width / 2, canvas.height / 2);

    await Promise.all(visualObjects.map(async vo => {
       if (vo.type === "ImageObject") {
           ctx.save();
           ctx.translate(vo.content.x, vo.content.y);
           console.log(vo.content.rotation);
           ctx.rotate(-vo.content.rotation);
           console.log(vo.content.image);
           let image = await loadImage("assets/" + vo.content.image);
           ctx.drawImage(image, -image.width / 2, -image.height / 2);
           ctx.restore();
       }
       else if (vo.type === "Line") {
           ctx.moveTo(vo.content.x1, vo.content.y1);
           ctx.lineTo(vo.content.x2, vo.content.y2);
           ctx.stroke();
       }
    }));
    ctx.restore();
}

rust
    .then(m => m.run(f))
    .catch(console.error)
;

const execute = () => {
    console.log(document.getElementById("code").value);
    rust
        .then(m => m.exec(document.getElementById("code").value))
        .catch(console.error);
}

window.onload = () => {
    let exec_button = document.getElementById("exec");
    exec_button.onclick = execute;
}
