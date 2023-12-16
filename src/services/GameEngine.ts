type Coordinate = {
  x: number;
  y: number;
};

type Pellet = {
  position: Coordinate;
  size: number;
  color: string;
  frame_count_offset: number;
};

type Snake = {
  bodies: Coordinate[];
  speed: number;
  color: string;
  velocity: Coordinate;
  frame_count_offset: number;
};

type Message = {
  isAlive: boolean;
  snakes: Snake[];
  pellets: Pellet[];
};

export default class GameEngine {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  offsetCanvas: HTMLCanvasElement;
  offsetCtx: CanvasRenderingContext2D;
  socket: WebSocket;
  mouseX: number = 0;
  mouseY: number = 0;

  constructor(
    canvas: HTMLCanvasElement,
    ctx: CanvasRenderingContext2D,
    uri: string
  ) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    addEventListener("resize", () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      this.socket.send(`w ${this.canvas.width} ${this.canvas.height}`);
    });

    this.canvas = canvas;
    this.ctx = ctx;
    this.offsetCanvas = document.createElement("canvas");
    this.offsetCanvas.width = canvas.width;
    this.offsetCanvas.height = canvas.height;
    this.offsetCtx = this.offsetCanvas.getContext("2d")!;
    this.drawBackground();

    this.socket = new WebSocket(uri);

    this.socket.onmessage = (e) => {
      const message: Message = JSON.parse(e.data);
      console.debug(message);
      this.update(message);
    };

    this.socket.onopen = () => {
      this.socket.send(`w ${this.canvas.width} ${this.canvas.height}`);
    };

    addEventListener("mousemove", (e) => {
      this.mouseX = e.clientX;
      this.mouseY = e.clientY;
    });
  }

  drawBackground() {
    // TODO: Change proper background
    // random constellation
    const stars = 100;
    this.offsetCtx.fillStyle = "white";
    for (let i = 0; i < stars; i++) {
      const x = Math.random() * this.canvas.width;
      const y = Math.random() * this.canvas.height;
      this.offsetCtx.beginPath();
      this.offsetCtx.arc(x, y, 1, 0, 2 * Math.PI);
      this.offsetCtx.fill();
      this.offsetCtx.closePath();
    }
  }

  drawSnake(snake: Snake) {
    this.ctx.fillStyle = snake.color;
    this.ctx.shadowColor = "none";
    this.ctx.shadowBlur = 0;
    // draw body
    for (let body of snake.bodies.reverse()) {
      const { x, y } = body;
      this.ctx.beginPath();
      this.ctx.arc(x, y, 15, 0, 2 * Math.PI);
      this.ctx.fill();
      this.ctx.closePath();
    }
    // draw eyes
    const head = snake.bodies[snake.bodies.length - 1];
    const close = 45;
    const radius = 7;
    const theta = Math.atan2(snake.velocity.y, snake.velocity.x);
    const eye1 = {
      x: head.x + Math.cos(theta - close) * radius,
      y: head.y + Math.sin(theta - close) * radius,
    };
    const eye2 = {
      x: head.x + Math.cos(theta + close) * radius,
      y: head.y + Math.sin(theta + close) * radius,
    };
    this.ctx.fillStyle = "white";
    this.ctx.beginPath();
    this.ctx.arc(eye1.x, eye1.y, 4, 0, 2 * Math.PI);
    this.ctx.fill();
    this.ctx.arc(eye2.x, eye2.y, 4, 0, 2 * Math.PI);
    this.ctx.fill();
    this.ctx.closePath();
    this.ctx.fillStyle = "black";
    this.ctx.beginPath();
    this.ctx.arc(eye1.x, eye1.y, 2, 0, 2 * Math.PI);
    this.ctx.fill();
    this.ctx.arc(eye2.x, eye2.y, 2, 0, 2 * Math.PI);
    this.ctx.fill();
    this.ctx.closePath();
  }

  drawPellet(pellet: Pellet) {
    const { x, y } = pellet.position;
    const size = pellet.size;
    const frame_count_offset = pellet.frame_count_offset;
    const h = pellet.color;
    const l = Math.abs(30 * Math.sin(frame_count_offset / 7)) + 50;
    const s = 100;
    this.ctx.beginPath();
    this.ctx.fillStyle = `hsl(${h}, ${s}%, ${l}%)`;
    this.ctx.shadowColor = `hsl(${h}, ${s}%, ${l}%)`;
    this.ctx.shadowBlur = size * 10;
    this.ctx.arc(x, y, size * 2, 0, 2 * Math.PI);
    this.ctx.fill();
    this.ctx.closePath();
  }

  update(message: Message) {
    // update velocity
    const dx = this.mouseX - this.canvas.width / 2;
    const dy = this.mouseY - this.canvas.height / 2;
    const angle = Math.atan2(dy, dx);
    const vx = Math.cos(angle);
    const vy = Math.sin(angle);
    this.socket.send(`v ${vx} ${vy}`);

    // update canvas
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.ctx.drawImage(this.offsetCanvas, 0, 0);
    for (let snake of message.snakes) {
      this.drawSnake(snake);
    }
    for (let pellet of message.pellets) {
      this.drawPellet(pellet);
    }
  }
}
