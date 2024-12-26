
import javax.swing.JFrame;

public class Principal {

	public static void main(String[] args) {
		JFrame ventana = new JFrame();
		ventana.setResizable(false);
		crearGUI(ventana);
	}
	
	public static void crearGUI( JFrame ventana ) {
		Panel panel = new Panel();
		Controlador controlador = new Controlador(panel);
		panel.controlador(controlador);
		ventana.setContentPane(panel);
		ventana.pack();
		ventana.setVisible(true);
		ventana.setDefaultCloseOperation( JFrame.EXIT_ON_CLOSE );	// Cuando se cierra la ventana se cierra la aplicación
	}
}
